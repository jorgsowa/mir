===description===
Only interfaces can be implemented or extended by an interface; a class, trait or enum there is a compile error.
===file===
<?php
class Shape {}
trait Named {}
enum Suit {
    case Hearts;
}

class Square implements Shape {}
//                      ^^^^^ CompileError: Square cannot implement Shape - it is not an interface [implements-non-interface]

class Label implements Named {}
//                     ^^^^^ CompileError: Label cannot implement Named - it is not an interface [implements-non-interface]

class Card implements Suit {}
//                    ^^^^ CompileError: Card cannot implement Suit - it is not an interface [implements-non-interface]

interface Drawable extends Shape {}
//                         ^^^^^ CompileError: Drawable cannot implement Shape - it is not an interface [implements-non-interface]
