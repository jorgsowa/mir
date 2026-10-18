===description===
A constructor's signature is unrelated to its parent's, so it never inherits
the parent constructor's `@param` refinement by position: a child taking
`Throwable|string` keeps it instead of collapsing to the parent's `string`.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Base {
    /** @param non-empty-string $message */
    public function __construct($message = 'x') {}
}

final class Child extends Base {
    public function __construct(Throwable|string $exception) {
        /** @mir-check $exception is Throwable|string */
        parent::__construct($exception instanceof Throwable ? $exception->getMessage() : $exception);
    }
}

final class Failure extends RuntimeException {
    public function __construct(Throwable|string $exception) {
        /** @mir-check $exception is Throwable|string */
        parent::__construct(
            $exception instanceof Throwable ? $exception->getMessage() : $exception,
            0,
            $exception instanceof Throwable ? $exception : null,
        );
    }
}

final class Missing extends LogicException {
    public function __construct(?string $message = null) {
        if (null === $message) {
            $message = 'missing';
        }
        parent::__construct($message);
    }
}
