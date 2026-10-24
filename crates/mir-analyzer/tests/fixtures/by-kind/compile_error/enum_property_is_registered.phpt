===description===
A property in an enum is a compile error; it is still registered, so reads of it keep its type and are not undefined. Redeclaring the built-in `name` adds nothing more.
===file===
<?php
enum E {
    case A;
    public int $x = 1;
//             ^^ CompileError: Compile error: Enum E cannot include properties [enum-property]
    /** @var list<string> */
    public static array $names = [];
//                      ^^^^^^ CompileError: Compile error: Enum E cannot include properties [enum-property]
    public string $name;
//                ^^^^^ CompileError: Compile error: Enum E cannot include properties [enum-property]
    public function f(): int { return $this->x; }
    public static function g(): string { return self::$names[0] ?? ""; }
    public function h(): int { return $this->missing; }
//                             ^^^^^^^^^^^^^^^^^^^^^^ MixedReturnStatement: Cannot return a mixed type from function with declared return type 'int'
//                                           ^^^^^^^ UndefinedProperty: Property E::$missing does not exist
}
