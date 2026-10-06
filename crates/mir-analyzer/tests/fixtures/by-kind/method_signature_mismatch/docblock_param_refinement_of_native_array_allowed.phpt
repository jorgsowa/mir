===description===
A docblock narrowing of a native array param is not an override violation; an incompatible or native narrowing still is.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Base {
    protected function shape(array $p): void {}
    protected function nullableShape(?array $p): void {}
    /** @param list<int> $p */
    protected function intList(array $p): void {}
    /** @param list<int> $p */
    protected function incompatible(array $p): void {}
    protected function nativeNarrow(array|string $p): void {}
}

class Child extends Base {
    /** @psalm-param array{x: int} $p */
    protected function shape(array $p): void {}

    /** @param array{x: int}|null $p */
    protected function nullableShape(?array $p): void {}

    /** @param non-empty-list<int> $p */
    protected function intList(array $p): void {}

    /** @param list<string> $p */
    protected function incompatible(array $p): void {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MethodSignatureMismatch: Method Child::incompatible() signature mismatch: parameter $p type 'list<string>' is incompatible with parent type 'list<int>'

    protected function nativeNarrow(array $p): void {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MethodSignatureMismatch: Method Child::nativenarrow() signature mismatch: parameter $p type 'array' is narrower than parent type 'array|string'
}

interface Shaper {
    /** @param list<int> $p */
    public function put(array $p): void;
}

class Impl implements Shaper {
    /** @param non-empty-list<int> $p */
    public function put(array $p): void {}
}
