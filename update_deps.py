import re

with open("docs/dependency-graph.dot", "r") as f:
    content = f.read()

# Replace Rust Core
content = content.replace('    main      [label="main.rs\\n(entry)", shape=box, style=filled, fillcolor="#c9d6ff"];', 
                          '    main      [label="main.rs\\n(entry)", shape=box, style=filled, fillcolor="#c9d6ff"];\n    api_gateway [label="api_gateway/main.rs\\n(Axum+gRPC)", shape=box, style=filled, fillcolor="#ffb8b8"];\n    ai_bridge [label="ai_bridge.rs\\n(gRPC Client)", shape=box, style=filled, fillcolor="#ffb8b8"];')

# Add AI Node Python
content = content.replace('  subgraph cluster_python {', 
                          '  subgraph cluster_python {\n    ai_node [label="ai_node/main.py\\n(PyTorch + gRPC)", shape=box, style=filled, fillcolor="#d2b4de"];\n    fno_pinn [label="models/fno.py, pinn.py\\n(Deep Learning)", shape=box, style=filled, fillcolor="#d2b4de"];')

# Add PyTorch requirements
content = content.replace('    numpy [label="numpy"];', 
                          '    numpy [label="numpy"];\n    torch [label="torch"];\n    grpcio [label="grpcio"];')

# Add Edges for gRPC
content = content.replace('  // external crates (used by core + tools)', 
                          '  // Microservices / gRPC / AI Flow\n  advphys -> ai_bridge;\n  ai_bridge -> api_gateway [label="HTTP GET /test_inference"];\n  api_gateway -> ai_node [label="gRPC (Tonic -> grpcio)"];\n  ai_node -> fno_pinn [label="Tensors"];\n\n  // external crates (used by core + tools)')

# Add missing Tonic/Axum
content = content.replace('    reqwest [label="reqwest 0.12"];', 
                          '    reqwest [label="reqwest 0.12"];\n    axum [label="axum 0.8"];\n    tonic [label="tonic 0.10"];\n    prost [label="prost"];')

with open("docs/dependency-graph.dot", "w") as f:
    f.write(content)
