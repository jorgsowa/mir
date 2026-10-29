===description===
PHP reports only the first of a set hook's parameter errors: count, then by-reference, variadic, default value, type.
===config===
<mir>
  <phpVersion>8.4</phpVersion>
</mir>
===file===
<?php
class Box {
    public int $count { set($a, $b) {} }
//                         ^ CompileError: set hook of property Box::$count must accept exactly one parameters [set-hook-parameter-count]
    public int $countByRef { set(string &$a, $b) {} }
//                              ^ CompileError: set hook of property Box::$countByRef must accept exactly one parameters [set-hook-parameter-count]
    public int $byRef { set(string &$a) {} }
//                          ^^^^^^^^^^ CompileError: Parameter $a of set hook Box::$byRef must not be pass-by-reference [set-hook-parameter-by-reference]
    public int $byRefVariadic { set(int &...$a) {} }
//                                  ^^^^^^^^^^ CompileError: Parameter $a of set hook Box::$byRefVariadic must not be pass-by-reference [set-hook-parameter-by-reference]
    public int $byRefDefault { set(int &$a = 1) {} }
//                                 ^^^^^^^^^^^ CompileError: Parameter $a of set hook Box::$byRefDefault must not be pass-by-reference [set-hook-parameter-by-reference]
    public int $variadic { set(string ...$a) {} }
//                             ^^^^^^^^^^^^ CompileError: Parameter $a of set hook Box::$variadic must not be variadic [set-hook-parameter-variadic]
    public int $default { set($a = 1) {} }
//                            ^^^^^^ CompileError: Parameter $a of set hook Box::$default must not have a default value [set-hook-parameter-default]
    public int $type { set(string $a) {} }
//                         ^^^^^^^^^ CompileError: Type of parameter $a of hook Box::$type::set must be compatible with property type [set-hook-parameter-type]
    public $untyped { set(int $a) {} }
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MissingPropertyType: Property Box::$untyped has no type annotation
//                        ^^^^^^ CompileError: Type of parameter $a of hook Box::$untyped::set must be compatible with property type [set-hook-parameter-type]
    public $untypedCount { set(int $a, $b) {} }
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MissingPropertyType: Property Box::$untypedCount has no type annotation
//                            ^ CompileError: set hook of property Box::$untypedCount must accept exactly one parameters [set-hook-parameter-count]
    public int $valid { set(int $a) {} }

    public int $outer {
        set(int $a) {
            new class {
                public int $inner { set(&$v) {} }
//                                      ^^^ CompileError: Parameter $v of set hook class@anonymous::$inner must not be pass-by-reference [set-hook-parameter-by-reference]
            };
        }
    }

    public int $outerBad {
        set(&$a) {
//          ^^^ CompileError: Parameter $a of set hook Box::$outerBad must not be pass-by-reference [set-hook-parameter-by-reference]
            new class {
                public int $inner { set(...$v) {} }
//                                      ^^^^^ CompileError: Parameter $v of set hook class@anonymous::$inner must not be variadic [set-hook-parameter-variadic]
            };
        }
    }
}

final class Promoted {
    public function __construct(
        public int $promoted { set(string $a, $b) {} },
//                                ^ CompileError: set hook of property Promoted::$promoted must accept exactly one parameters [set-hook-parameter-count]
    ) {}
}
