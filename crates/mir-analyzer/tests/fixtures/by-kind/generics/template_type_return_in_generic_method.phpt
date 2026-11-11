===description===
Inside a generic method, a `template-type<Box<T>, Box, 'T'>` return type is the template `T`, so returning the box's `T` value is not `MixedReturnStatement`.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
declare(strict_types=1);

/** @template T */
final class Box {
    /** @param T $value */
    public function __construct(public mixed $value) {}
}

final class Unwrapper {
    /**
     * @template T
     * @param Box<T> $box
     * @return template-type<Box<T>, Box, 'T'>
     */
    public function unwrap(Box $box): mixed {
        return $box->value;
    }

    /**
     * @param Box<int> $box
     * @return template-type<Box<int>, Box, 'T'>
     */
    public function unwrapInt(Box $box): int {
        return 'text';
//      ^^^^^^^^^^^^^^ InvalidReturnType: Return type '"text"' is not compatible with declared 'int'
    }
}

function takesString(string $value): void {}

takesString((new Unwrapper())->unwrap(new Box('text')));
