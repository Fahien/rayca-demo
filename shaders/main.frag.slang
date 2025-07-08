// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

import vertex;

[vk::binding(0, 2)]
ConstantBuffer<float4> material_color;
[vk::binding(1, 2)]
Texture2D texture;

[shader("fragment")]
float4 main(in Vertex vertex) : SV_Target {
    SamplerState s;
    return vertex.color * material_color * texture.Sample(s, vertex.uv);
}
