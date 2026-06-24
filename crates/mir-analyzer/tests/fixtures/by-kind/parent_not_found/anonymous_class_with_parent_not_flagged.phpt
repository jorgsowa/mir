===description===
ParentNotFound does NOT fire when parent:: is used inside an anonymous class
that has an extends clause.
===config===
suppress=UnusedVariable,MissingParamType,MissingReturnType,MissingConstructor
===file===
<?php

class Base {
    public function greet(): string { return 'hello'; }
    protected function compute(): int { return 42; }
}

// parent:: instance method call
$a = new class extends Base {
    public function greet(): string {
        return parent::greet() . '!';
    }

    public function run(): int {
        return parent::compute();
    }
};

// anonymous class in function scope also works
function make(): object {
    return new class extends Base {
        public function greet(): string {
            return '[' . parent::greet() . ']';
        }
    };
}
===expect===
