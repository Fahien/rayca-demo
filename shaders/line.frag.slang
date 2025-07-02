// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

import vertex;

[shader("fragment")]
float4 main(in Vertex vertex) : SV_Target {
    return vertex.color;
}
