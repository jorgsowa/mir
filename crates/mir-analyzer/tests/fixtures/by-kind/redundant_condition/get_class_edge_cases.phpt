===description===
get_class() narrowing edge cases with strict equality
===config===
suppress=MissingReturnType,MixedArgument
===file===
<?php
namespace App;

class Foo {
    public function foo() {}
}

class Bar {
    public function bar() {}
}

class Parent_ {
    public function parentMethod() {}
}

class Child extends Parent_ {
    public function childMethod() {}
}

// Case 1: Namespace - with full FQCN
function testNamespacedFQCN(object $obj) {
    if (get_class($obj) === 'App\\Foo') {
        $obj->foo();
    }
}

// Case 2: Mixed type narrowing
function testMixedToSpecific(mixed $obj) {
    if (get_class($obj) === 'App\\Foo') {
        $obj->foo();
    }
}

// Case 3: Multiple variables with AND
function testMultipleVariables(object $obj, object $obj2) {
    if (get_class($obj) === 'App\\Foo' && get_class($obj2) === 'App\\Bar') {
        $obj->foo();
        $obj2->bar();
    }
}

// Case 4: get_class with unrelated AND condition
function testUnrelatedAND(Foo|Bar $obj) {
    if (get_class($obj) === 'App\\Foo' && strlen('test') > 0) {
        $obj->foo();
    }
}

// Case 5: Union type narrowing to first type
function testUnionFirst(Foo|Bar $obj) {
    if (get_class($obj) === 'App\\Foo') {
        $obj->foo();
    }
}

// Case 6: Union type narrowing to second type
function testUnionSecond(Foo|Bar $obj) {
    if (get_class($obj) === 'App\\Bar') {
        $obj->bar();
    }
}

// Case 7: Invalid class name (string exists, class doesn't)
function testNonExistentClass(object $obj) {
    if (get_class($obj) === 'App\\NonExistent') {
        // $obj narrowed to App\NonExistent (even though class doesn't exist)
    }
}

// Case 8: With null coalescing (should not affect narrowing)
function testWithNullCheck(object|null $obj) {
    if (($obj ?? new Foo()) instanceof Foo && get_class($obj ?? new Foo()) === 'App\\Foo') {
        if ($obj !== null) {
            $obj->foo();
        }
    }
}
===expect===
