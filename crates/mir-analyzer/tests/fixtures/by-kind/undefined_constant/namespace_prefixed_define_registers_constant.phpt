===description===
`define(__NAMESPACE__ . '\NAME', …)` declares the constant in the current
namespace, with the value's type. Another name in that namespace stays undefined.
===file===
<?php
namespace App\Internal;

\define(__NAMESPACE__ . '\TIMEOUT', 110);

function relative(): int { return TIMEOUT; }
function qualified(): int { return \App\Internal\TIMEOUT; }
function missing(): void { echo OTHER; }
//                              ^^^^^ UndefinedConstant: Constant OTHER is not defined
