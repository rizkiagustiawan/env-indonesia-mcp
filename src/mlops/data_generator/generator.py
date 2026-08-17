import os
import torch
import httpx
import asyncio

async def fetch_synthetic_data(client, mock=False):
    if mock:
        # Mock physical output for testing
        return {"input_h": [1.0]*4, "output_h": [1.05]*4, "width": 2, "height": 2}
        
    try:
        response = await client.get("http://127.0.0.1:3000/test_inference")
        response.raise_for_status()
        # In a real scenario, this would be a full grid response. 
        # For now, we mock the format based on the gateway's current response.
        data = response.json()
        return {"input_h": [1.0]*4, "output_h": [data.get("predicted_depth_sample", 1.05)]*4, "width": 2, "height": 2}
    except Exception as e:
        print(f"Error fetching data: {e}")
        return None

async def generate_batch(num_samples=10, output_dir="src/mlops/.data", mock=False):
    os.makedirs(output_dir, exist_ok=True)
    
    async with httpx.AsyncClient(timeout=10.0) as client:
        tasks = [fetch_synthetic_data(client, mock) for _ in range(num_samples)]
        results = await asyncio.gather(*tasks)
        
        for i, res in enumerate(results):
            if res:
                # Convert to tensors
                input_tensor = torch.tensor(res["input_h"]).reshape(1, res["width"], res["height"], 1)
                output_tensor = torch.tensor(res["output_h"]).reshape(1, res["width"], res["height"], 1)
                
                # Save to disk
                torch.save({"x": input_tensor, "y": output_tensor}, os.path.join(output_dir, f"sample_{i}.pt"))

if __name__ == "__main__":
    asyncio.run(generate_batch(num_samples=5))
