===description===
False positive undefined class in callable
===config===
suppress=MissingReturnType,MixedArgument
===file===
<?php
// This demonstrates the FALSE POSITIVE issue:
// When a TLiteralString is resolved without class-string context,
// the analyzer incorrectly reports UndefinedClass

/**
 * @param callable $callback
 */
function array_map_wrapper(callable $callback, array $data) {
    return array_map($callback, $data);
}

// Using a function name as a string - should NOT emit UndefinedClass
array_map_wrapper("trim", ["  hello  ", "  world  "]);

// Using a method array callback - should NOT emit UndefinedClass for parts
class Processor {
    public function process($item) {
        return strtoupper($item);
    }
}

$processor = new Processor();
array_map_wrapper([$processor, "process"], ["a", "b"]);
===expect===
