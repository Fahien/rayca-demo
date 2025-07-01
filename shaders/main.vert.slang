// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

import vertex;

[shader("vertex")]
Vertex main(float4 pos, float4 color) {
    Vertex out;
    out.pos = pos;
    out.color = color;
    return out;
}
