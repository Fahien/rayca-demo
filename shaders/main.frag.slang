// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

import vertex;

[vk::binding(0, 2)]
ConstantBuffer<float4> color;
[vk::binding(1, 2)]
Texture2D albedo;
[vk::binding(1, 2)]
SamplerState s;

struct Output {
    float4 color : SV_Target0;
    float4 normal : SV_Target1;
}

[shader("fragment")]
Output main(in Vertex vertex) {
    Output o;
    o.color = vertex.color * color * albedo.Sample(s, vertex.uv);
    o.normal = (normalize(vertex.normal) + 1.0) / 2.0;
    return o;
}
