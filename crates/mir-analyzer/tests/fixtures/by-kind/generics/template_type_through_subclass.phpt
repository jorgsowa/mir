===description===
`template-type` follows `@extends`: a subclass that fixes the template yields the fixed argument.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
    <MixedReturnStatement errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
declare(strict_types=1);

/** @template T */
class Result {
    /** @param T $payload */
    public function __construct(public mixed $payload) {}
}

/** @extends Result<string> */
final class TextResult extends Result {}

/**
 * @template TResult of Result
 * @param TResult $result
 * @return template-type<TResult, Result, 'T'>
 */
function payload(Result $result): mixed {
    return $result->payload;
}

function takesInt(int $value): void {}
function takesString(string $value): void {}

takesString(payload(new TextResult('ok')));
takesInt(payload(new TextResult('ok')));
//       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $value of takesInt() expects 'int', got 'string'
