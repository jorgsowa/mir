===description===
FN: unary `-` never checked for a non-numeric operand, unlike binary
arithmetic and unary `~`.
===config===
suppress=UnusedVariable
===file===
<?php
enum Suit {
    case Hearts;
}
$a = -Suit::Hearts;
===expect===
InvalidOperand@5:6-5:18: Operator '-' not supported between 'Suit' and ''
