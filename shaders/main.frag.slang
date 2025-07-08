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

[shader("fragment")]
float4 main(in Vertex vertex) : SV_Target
{
    return vertex.color * color * albedo.Sample(s, vertex.uv);
}
