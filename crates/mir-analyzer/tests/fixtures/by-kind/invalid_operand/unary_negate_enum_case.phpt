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
//    ^^^^^^^^^^^^ InvalidOperand: Operator '-' not supported for operand of type 'Suit'
===expect===
