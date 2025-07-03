// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

import vertex;

[vk::binding(0, 2)]
Texture2D texture;
[vk::binding(0, 2)]
SamplerState sampler;

[shader("fragment")]
float4 main(in Vertex vertex) : SV_Target
{
    return vertex.color * texture.Sample(sampler, vertex.uv);
}
