===description===
A class cannot extend an enum or a trait; no InvalidExtendClass or interface cascades follow.
===file===
<?php
enum Suit {
    case Hearts;
}

trait Greets {}

class Card extends Suit {}
//                 ^^^^ CompileError: Class Card cannot extend enum Suit [extends-enum]

class Greeter extends Greets {}
//                    ^^^^^^ CompileError: Class Greeter cannot extend trait Greets [extends-trait]
