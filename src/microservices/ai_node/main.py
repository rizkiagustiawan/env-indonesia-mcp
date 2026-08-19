import grpc
from concurrent import futures
import time
import torch
import numpy as np
import logging
import sys
import os

sys.path.append(os.path.join(os.path.dirname(__file__), 'proto_gen'))

import ai_inference_pb2
import ai_inference_pb2_grpc
from models.fno import FNO2d
from models.pinn import GroundwaterPINN
from models.pino_swe import PINOSWE2D

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger("ai_node")

class InferenceEngineServicer(ai_inference_pb2_grpc.InferenceEngineServicer):
    def __init__(self):
        self.device = torch.device('cuda' if torch.cuda.is_available() else 'cpu')
        
        # Initialize FNO for flood/SWE macro-scale
        logger.info(f"Loading FNO model on {self.device}")
        self.fno = FNO2d(modes1=8, modes2=8, width=20).to(self.device)
        self.fno.eval() # inference mode
        
        # Initialize PINN for groundwater micro-scale
        logger.info(f"Loading PINN model on {self.device}")
        self.pinn = GroundwaterPINN().to(self.device)
        self.pinn.eval()
        
        # Initialize PINO for shallow water
        logger.info(f"Loading PINO SWE model on {self.device}")
        self.pino = PINOSWE2D(modes1=12, modes2=12, width=32).to(self.device)
        self.pino.eval()

    def PredictFloodFNO(self, request, context):
        start_time = time.time()
        
        try:
            width = request.width
            height = request.height
            # Shape: [batch=1, width, height, channels=1]
            # Since FNO isn't modified yet for DEM channels, we just pass initial_h
            initial_h = np.array(request.initial_h).reshape((1, width, height, 1))
            
            # Convert to tensor and pass through FNO
            x_tensor = torch.tensor(initial_h, dtype=torch.float32).to(self.device)
            
            with torch.no_grad():
                pred_tensor = self.fno(x_tensor)
                
            predicted_h = pred_tensor.cpu().numpy().flatten()
            
            response = ai_inference_pb2.FloodResponse(
                predicted_h=predicted_h.tolist(),
                width=width,
                height=height,
                inference_ms=(time.time() - start_time) * 1000.0
            )
            return response
            
        except Exception as e:
            logger.error(f"FNO Inference failed: {e}")
            context.set_code(grpc.StatusCode.INTERNAL)
            context.set_details(str(e))
            return ai_inference_pb2.FloodResponse()

    def PredictShallowWaterPINO(self, request, context):
        start_time = time.time()
        
        try:
            width = request.width
            height = request.height
            
            # Reshape inputs
            initial_h = np.array(request.initial_h).reshape((width, height))
            initial_u = np.array(request.initial_u).reshape((width, height))
            initial_v = np.array(request.initial_v).reshape((width, height))
            dem = np.array(request.dem).reshape((width, height)) if len(request.dem) > 0 else np.zeros((width, height))
            
            # Stack to [batch=1, channels=4, width, height] (added dem)
            x_input = np.stack([initial_h, initial_u, initial_v, dem], axis=0)
            x_input = np.expand_dims(x_input, axis=0)
            
            x_tensor = torch.tensor(x_input, dtype=torch.float32).to(self.device)
            
            with torch.no_grad():
                pred_tensor = self.pino(x_tensor)
                
            # Extract predictions
            pred_np = pred_tensor.cpu().numpy()[0] # shape: [3, width, height]
            pred_h = pred_np[0].flatten().tolist()
            pred_u = pred_np[1].flatten().tolist()
            pred_v = pred_np[2].flatten().tolist()
            
            response = ai_inference_pb2.ShallowWaterResponse(
                predicted_h=pred_h,
                predicted_u=pred_u,
                predicted_v=pred_v,
                width=width,
                height=height,
                inference_ms=(time.time() - start_time) * 1000.0
            )
            return response
            
        except Exception as e:
            logger.error(f"PINO SWE Inference failed: {e}")
            context.set_code(grpc.StatusCode.INTERNAL)
            context.set_details(str(e))
            return ai_inference_pb2.ShallowWaterResponse()

    def PredictGroundwaterPINN(self, request, context):
        start_time = time.time()
        
        try:
            initial_c = np.array(request.initial_concentration)
            
            # For a PINN inference we usually pass [t, x] coordinates.
            # Here we mock the spatial grid creation for a 1D problem and predict at t=t_end
            n_points = len(initial_c)
            x_spatial = np.linspace(0, 1, n_points)
            t_eval = np.ones(n_points) * request.t_end
            
            # Input shape: [n_points, 2] where 2 is (t, x)
            inputs = np.stack([t_eval, x_spatial], axis=1)
            x_tensor = torch.tensor(inputs, dtype=torch.float32).to(self.device)
            
            with torch.no_grad():
                pred_tensor = self.pinn(x_tensor)
                
            predicted_c = pred_tensor.cpu().numpy().flatten()
            
            response = ai_inference_pb2.GroundwaterResponse(
                predicted_concentration=predicted_c.tolist(),
                inference_ms=(time.time() - start_time) * 1000.0
            )
            return response
            
        except Exception as e:
            logger.error(f"PINN Inference failed: {e}")
            context.set_code(grpc.StatusCode.INTERNAL)
            context.set_details(str(e))
            return ai_inference_pb2.GroundwaterResponse()

def serve():
    server = grpc.server(futures.ThreadPoolExecutor(max_workers=10))
    ai_inference_pb2_grpc.add_InferenceEngineServicer_to_server(InferenceEngineServicer(), server)
    server.add_insecure_port('[::]:50051')
    server.start()
    logger.info("AI Inference Node (PyTorch+gRPC) listening on port 50051")
    server.wait_for_termination()

if __name__ == '__main__':
    serve()
