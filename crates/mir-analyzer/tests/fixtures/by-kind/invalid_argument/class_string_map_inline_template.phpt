===description===
`class-string-map<T of Bound, T>` declares its template in place; the value is read as the bound.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <MissingParamType errorLevel="suppress"/>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace App;

/** @param class-string-map<T of \Throwable, T> $map */
function acceptsThrowableMap(array $map): void
{
    /** @mir-check $map is array<class-string, Throwable> */
    echo \count($map);
}

acceptsThrowableMap([\RuntimeException::class => new \RuntimeException()]);
acceptsThrowableMap([\RuntimeException::class => new \stdClass()]);
//                  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $map of acceptsThrowableMap() expects 'array<class-string, Throwable>', got 'non-empty-array<class-string<RuntimeException>, stdClass>'
