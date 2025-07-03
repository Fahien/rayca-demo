// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

import vertex;

ConstantBuffer<float4x4> model;

[vk::binding(0, 1)]
ConstantBuffer<float4x4> view;
[vk::binding(1, 1)]
ConstantBuffer<float4x4> proj;

[shader("vertex")]
Vertex main(float4 pos, float4 color) {
    Vertex out;
    out.pos = mul(proj, mul(view, mul(model, pos)));
    out.color = color;
    return out;
}
