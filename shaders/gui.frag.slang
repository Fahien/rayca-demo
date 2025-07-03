// Copyright © 2021-2024
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

import vertex;

[vk::binding(0, 0)]
Texture2D font_image;
[vk::binding(0, 0)]
SamplerState sampler;

[shader("fragment")]
float4 main(in Vertex vertex) : SV_Target
{
    return vertex.color * font_image.Sample(sampler, vertex.uv);
}
