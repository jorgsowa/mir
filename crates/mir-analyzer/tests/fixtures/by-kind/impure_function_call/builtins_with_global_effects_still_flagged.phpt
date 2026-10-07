===description===
Builtins that may run user code (autoloaders, iterators) or change process
state stay impure in a @pure function.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class Probe {
    /**
     * @pure
     * @param Traversable<int, int> $items
     * @param class-string $class
     */
    public static function inspect(Traversable $items, string $class, object $value): bool {
        $copy = iterator_to_array($items);
//              ^^^^^^^^^^^^^^^^^^^^^^^^^ ImpureFunctionCall: Calling impure function iterator_to_array() in a @pure function
        $locale = setlocale(LC_CTYPE, '0');
//                ^^^^^^^^^^^^^^^^^^^^^^^^ ImpureFunctionCall: Calling impure function setlocale() in a @pure function
        $isEnum = enum_exists($value::class);
//                ^^^^^^^^^^^^^^^^^^^^^^^^^^ ImpureFunctionCall: Calling impure function enum_exists() in a @pure function
        $property = new ReflectionProperty($value, 'id');
//                  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ ImpureFunctionCall: Calling impure function ReflectionProperty::__construct() in a @pure function
        $named = is_callable($class);
//               ^^^^^^^^^^^^^^^^^^^ ImpureFunctionCall: Calling impure function is_callable() in a @pure function
        return class_exists($class);
//             ^^^^^^^^^^^^^^^^^^^^ ImpureFunctionCall: Calling impure function class_exists() in a @pure function
    }
}
