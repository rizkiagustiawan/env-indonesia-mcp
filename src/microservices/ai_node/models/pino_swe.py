import torch
import torch.nn as nn
import torch.nn.functional as F

class SpectralConv2d(nn.Module):
    def __init__(self, in_channels, out_channels, modes1, modes2):
        super(SpectralConv2d, self).__init__()
        self.in_channels = in_channels
        self.out_channels = out_channels
        self.modes1 = modes1
        self.modes2 = modes2

        self.scale = (1 / (in_channels * out_channels))
        self.weights1 = nn.Parameter(self.scale * torch.rand(in_channels, out_channels, self.modes1, self.modes2, dtype=torch.cfloat))
        self.weights2 = nn.Parameter(self.scale * torch.rand(in_channels, out_channels, self.modes1, self.modes2, dtype=torch.cfloat))

    def forward(self, x):
        batchsize = x.shape[0]
        # Compute Fourier coeffcients up to factor of e^(- something constant)
        x_ft = torch.fft.rfft2(x)
        
        # Ensure we don't try to access more modes than the input size allows
        m1 = min(self.modes1, x.size(-2))
        m2 = min(self.modes2, x.size(-1)//2 + 1)
        
        # Multiply relevant Fourier modes
        out_ft = torch.zeros(batchsize, self.out_channels,  x.size(-2), x.size(-1)//2 + 1, dtype=torch.cfloat, device=x.device)
        
        # complex multiplication
        # [batch, in_channels, x, y] * [in_channels, out_channels, x, y] -> [batch, out_channels, x, y]
        # Only use up to m1, m2 from the weights
        out_ft[:, :, :m1, :m2] = \
            torch.einsum("bixy,ioxy->boxy", x_ft[:, :, :m1, :m2], self.weights1[:, :, :m1, :m2])
        out_ft[:, :, -m1:, :m2] = \
            torch.einsum("bixy,ioxy->boxy", x_ft[:, :, -m1:, :m2], self.weights2[:, :, :m1, :m2])

        # Return to physical space
        x = torch.fft.irfft2(out_ft, s=(x.size(-2), x.size(-1)))
        return x

class PINOSWE2D(nn.Module):
    """
    Physics-Informed Neural Operator (PINO) for 2D Shallow Water Equations.
    Input: [batch, 3, nx, ny] - (h_initial, u_initial, v_initial)
    Output: [batch, 3, nx, ny] - (h_pred, u_pred, v_pred) at t + dt
    """
    def __init__(self, modes1=12, modes2=12, width=32):
        super(PINOSWE2D, self).__init__()
        self.modes1 = modes1
        self.modes2 = modes2
        self.width = width
        
        # 3 input channels (h, u, v), plus grid coords (x, y) = 5
        self.p = nn.Linear(5, self.width)
        
        # Re-initialize convolutions with dynamic modes per forward pass
        self.conv0 = SpectralConv2d(self.width, self.width, self.modes1, self.modes2)
        self.conv1 = SpectralConv2d(self.width, self.width, self.modes1, self.modes2)
        self.conv2 = SpectralConv2d(self.width, self.width, self.modes1, self.modes2)
        self.conv3 = SpectralConv2d(self.width, self.width, self.modes1, self.modes2)
        
        self.w0 = nn.Conv2d(self.width, self.width, 1)
        self.w1 = nn.Conv2d(self.width, self.width, 1)
        self.w2 = nn.Conv2d(self.width, self.width, 1)
        self.w3 = nn.Conv2d(self.width, self.width, 1)
        
        self.q = nn.Linear(self.width, 128)
        self.out = nn.Linear(128, 3) # Output 3 channels: h, u, v

    def get_grid(self, shape, device):
        batchsize, size_x, size_y = shape[0], shape[1], shape[2]
        gridx = torch.tensor(torch.linspace(0, 1, size_x), dtype=torch.float)
        gridx = gridx.reshape(1, size_x, 1, 1).repeat([batchsize, 1, size_y, 1])
        gridy = torch.tensor(torch.linspace(0, 1, size_y), dtype=torch.float)
        gridy = gridy.reshape(1, 1, size_y, 1).repeat([batchsize, size_x, 1, 1])
        return torch.cat((gridx, gridy), dim=-1).to(device)

    def forward(self, x):
        # x: [batch, 3, nx, ny]
        
        # Adjust modes to be at most nx//2 and ny//2
        orig_modes1 = self.modes1
        orig_modes2 = self.modes2
        
        nx, ny = x.shape[2], x.shape[3]
        max_mode1 = min(self.modes1, nx // 2 + 1)
        max_mode2 = min(self.modes2, ny // 2 + 1)
        
        self.conv0.modes1 = max_mode1
        self.conv0.modes2 = max_mode2
        self.conv1.modes1 = max_mode1
        self.conv1.modes2 = max_mode2
        self.conv2.modes1 = max_mode1
        self.conv2.modes2 = max_mode2
        self.conv3.modes1 = max_mode1
        self.conv3.modes2 = max_mode2
        
        grid = self.get_grid((x.shape[0], x.shape[2], x.shape[3]), x.device)
        
        # Permute x to [batch, nx, ny, channels] to concat with grid
        x = x.permute(0, 2, 3, 1)
        x = torch.cat((x, grid), dim=-1)
        
        x = self.p(x)
        x = x.permute(0, 3, 1, 2)
        
        x1 = self.conv0(x)
        x2 = self.w0(x)
        x = x1 + x2
        x = F.gelu(x)
        
        x1 = self.conv1(x)
        x2 = self.w1(x)
        x = x1 + x2
        x = F.gelu(x)
        
        x1 = self.conv2(x)
        x2 = self.w2(x)
        x = x1 + x2
        x = F.gelu(x)
        
        x1 = self.conv3(x)
        x2 = self.w3(x)
        x = x1 + x2
        
        x = x.permute(0, 2, 3, 1)
        x = self.q(x)
        x = F.gelu(x)
        x = self.out(x)
        
        # Return to [batch, channels, nx, ny]
        x = x.permute(0, 3, 1, 2)
        return x

    def physics_loss(self, h, u, v, h_t, u_t, v_t, h_x, u_x, v_x, h_y, u_y, v_y, z_x, z_y, g=9.81):
        """
        SWE Residuals for Physics-Informed regularization.
        Augmented with Topography (z) for Well-Balanced property.
        Using eta (water surface elevation) = h + z
        """
        # Wetting-drying threshold mask to avoid division by zero and singularity in dry cells
        h_threshold = 1e-3
        wet_mask = (h > h_threshold).float()

        # Mass conservation: h_t + (hu)_x + (hv)_y = 0
        eq1 = h_t + (h * u_x + u * h_x) + (h * v_y + v * h_y)
        
        # Momentum x: u_t + uu_x + vu_y + g(h_x + z_x) = 0
        eq2 = u_t + u * u_x + v * u_y + g * (h_x + z_x)
        
        # Momentum y: v_t + uv_x + vv_y + g(h_y + z_y) = 0
        eq3 = v_t + u * v_x + v * v_y + g * (h_y + z_y)
        
        # Apply mask so we don't penalize physics residuals on dry land
        eq1 = eq1 * wet_mask
        eq2 = eq2 * wet_mask
        eq3 = eq3 * wet_mask

        return torch.mean(eq1**2) + torch.mean(eq2**2) + torch.mean(eq3**2)
