===description===
The false branch of `method_exists($mixed, ...)` may still be a class-name string
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function f(mixed $c): string {
    if (method_exists($c, '__invoke')) {
        return 'invokable';
    } elseif (!\is_string($c)) {
        return '';
    } elseif (str_contains($c, '::')) {
        return 'static';
    }
    return $c;
}
