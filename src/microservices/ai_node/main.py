import grpc
from concurrent import futures
import time
import torch
import numpy as np
import logging
import sys
import os

# Add generated proto to path
sys.path.append(os.path.join(os.path.dirname(__file__), 'proto_gen'))

import ai_inference_pb2
import ai_inference_pb2_grpc

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger("ai_node")

class InferenceEngineServicer(ai_inference_pb2_grpc.InferenceEngineServicer):
    def PredictFloodFNO(self, request, context):
        start_time = time.time()
        logger.info(f"Received FNO request for site: {request.site_id}, bbox: {request.bbox}")
        
        # Determine device
        device = torch.device('cuda' if torch.cuda.is_available() else 'cpu')
        
        # Reconstruct matrix
        try:
            # Here we would run the actual FNO. For now, simulate.
            width = request.width
            height = request.height
            initial_h = np.array(request.initial_h).reshape((width, height))
            
            # Simulated inference
            time.sleep(0.05) 
            predicted_h = initial_h * 1.05 # Mock physics step
            
            response = ai_inference_pb2.FloodResponse(
                predicted_h=predicted_h.flatten().tolist(),
                width=width,
                height=height,
                inference_ms=(time.time() - start_time) * 1000.0
            )
            return response
            
        except Exception as e:
            logger.error(f"Inference failed: {e}")
            context.set_code(grpc.StatusCode.INTERNAL)
            context.set_details(str(e))
            return ai_inference_pb2.FloodResponse()

    def PredictGroundwaterPINN(self, request, context):
        start_time = time.time()
        logger.info(f"Received PINN request for site: {request.site_id}")
        
        try:
            # Simulate PINN
            initial_c = np.array(request.initial_concentration)
            time.sleep(0.03)
            predicted_c = initial_c * 0.9 # Mock decay
            
            response = ai_inference_pb2.GroundwaterResponse(
                predicted_concentration=predicted_c.tolist(),
                inference_ms=(time.time() - start_time) * 1000.0
            )
            return response
        except Exception as e:
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
