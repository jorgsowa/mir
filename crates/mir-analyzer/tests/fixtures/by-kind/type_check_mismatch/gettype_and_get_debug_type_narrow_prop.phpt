===description===
gettype($this->prop) === 'literal' and get_debug_type($this->prop) ===
'literal'/Foo::class narrow the property the same way the plain-variable
receiver does.
===config===
suppress=UnusedVariable,UnusedParam,MixedArgument
===file===
<?php
class Foo {}

class HasGettypeProp {
    /** @var int|string */
    public mixed $x;

    public function testGettypeString(): void {
        if (gettype($this->x) === 'string') {
            /** @mir-check $this->x is string */
            $_ = $this->x;
        }
    }

    public function testGettypeIntegerReversed(): void {
        if ('integer' === gettype($this->x)) {
            /** @mir-check $this->x is int */
            $_ = $this->x;
        }
    }

    public function testGettypeNotString(): void {
        if (gettype($this->x) !== 'string') {
            /** @mir-check $this->x is int */
            $_ = $this->x;
        }
    }
}

class HasGetDebugTypeProp {
    /** @var Foo|string */
    public mixed $x;

    public function testGetDebugTypeString(): void {
        if (get_debug_type($this->x) === 'string') {
            /** @mir-check $this->x is string */
            $_ = $this->x;
        }
    }

    public function testGetDebugTypeClassLiteral(): void {
        if (get_debug_type($this->x) === 'Foo') {
            /** @mir-check $this->x is Foo */
            $_ = $this->x;
        }
    }

    public function testGetDebugTypeClassConst(): void {
        if (get_debug_type($this->x) === Foo::class) {
            /** @mir-check $this->x is Foo */
            $_ = $this->x;
        }
    }

    public function testGetDebugTypeClassConstReversed(): void {
        if (Foo::class === get_debug_type($this->x)) {
            /** @mir-check $this->x is Foo */
            $_ = $this->x;
        }
    }
}
===expect===
MissingConstructor@4:0-4:22: Class HasGettypeProp has uninitialized properties but no constructor
MissingConstructor@30:0-30:27: Class HasGetDebugTypeProp has uninitialized properties but no constructor
