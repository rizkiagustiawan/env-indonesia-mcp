import torch
import torch.nn as nn

class GroundwaterPINN(nn.Module):
    def __init__(self, layers=[2, 20, 20, 20, 20, 1]):
        super(GroundwaterPINN, self).__init__()
        self.activation = nn.Tanh()
        self.loss_function = nn.MSELoss(reduction ='mean')
        
        self.linears = nn.ModuleList([nn.Linear(layers[i], layers[i+1]) for i in range(len(layers)-1)])
        
        # Physics parameters (e.g. Diffusion coefficient, velocity)
        self.D = nn.Parameter(torch.tensor([0.01], requires_grad=True))
        self.v = nn.Parameter(torch.tensor([0.1], requires_grad=True))
        
        # Init
        for i in range(len(layers)-1):
            nn.init.xavier_normal_(self.linears[i].weight.data, gain=1.0)
            nn.init.zeros_(self.linears[i].bias.data)

    def forward(self, x):
        # x is concatenated [t, spatial_x]
        if torch.is_tensor(x) != True:         
            x = torch.from_numpy(x).float()
            
        a = x
        for i in range(len(self.linears)-1):
            z = self.linears[i](a)
            a = self.activation(z)
        a = self.linears[-1](a)
        return a # returns predicted concentration C(t, x)
        
    def physics_loss(self, x):
        # Calculates PDE residual: dC/dt + v*dC/dx - D*d^2C/dx^2 = 0
        x.requires_grad = True
        c = self.forward(x)
        
        c_x_t = torch.autograd.grad(c, x, grad_outputs=torch.ones_like(c), create_graph=True)[0]
        c_t = c_x_t[:, 0:1]
        c_x = c_x_t[:, 1:2]
        
        c_xx_tt = torch.autograd.grad(c_x, x, grad_outputs=torch.ones_like(c_x), create_graph=True)[0]
        c_xx = c_xx_tt[:, 1:2]
        
        f = c_t + self.v * c_x - self.D * c_xx
        return self.loss_function(f, torch.zeros_like(f))
