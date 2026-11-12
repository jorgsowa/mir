===description===
`class-string-map<T of X, T>` maps each class-string key to an instance of that class.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
declare(strict_types=1);

namespace App;

/** @param class-string-map<T of \Throwable, T> $map */
function acceptsThrowableMap(array $map): void {}

/** @param array<class-string<\Throwable>, \Throwable> $map */
function forward(array $map): void
{
    acceptsThrowableMap($map);
}

acceptsThrowableMap([\RuntimeException::class => new \RuntimeException()]);
acceptsThrowableMap([\LogicException::class => new \InvalidArgumentException()]);
acceptsThrowableMap([
    \LogicException::class => new \LogicException(),
    \RuntimeException::class => new \UnexpectedValueException(),
]);
acceptsThrowableMap([\RuntimeException::class => new \LogicException()]);
//                  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $map of acceptsThrowableMap() expects 'array<class-string<Throwable>, Throwable> with each value an instance of its key', got 'non-empty-array<class-string<RuntimeException>, LogicException>'
