===description===
A scalar docblock `@param` refinement next to `{@inheritDoc}` is intentional; without it, or with a native narrowing, it still flags.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface Sink {
    public function text(string $v): void;
    public function count(int $v): void;
    public function either(int|string $v): void;
    public function plain(string $v): void;
}

final class Impl implements Sink {
    /**
     * {@inheritDoc}
     *
     * @param non-empty-string $v
     */
    public function text(string $v): void {
        /** @mir-check $v is non-empty-string */
    }

    /**
     * {@inheritDoc}
     *
     * @param int<1, 5> $v
     */
    public function count(int $v): void {}

    /**
     * {@inheritDoc}
     */
    public function either(int $v): void {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MethodSignatureMismatch: Method Impl::either() signature mismatch: parameter $v type 'int' is narrower than parent type 'int|string'

    /** @param non-empty-string $v */
    public function plain(string $v): void {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MethodSignatureMismatch: Method Impl::plain() signature mismatch: parameter $v type 'non-empty-string' is narrower than parent type 'string'
}
