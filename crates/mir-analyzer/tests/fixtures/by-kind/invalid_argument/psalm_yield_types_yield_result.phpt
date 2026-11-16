===description===
`$x = yield $expr` takes the `@psalm-yield` type of `$expr`'s class or an ancestor,
resolved against the object's template args; other yields stay mixed.
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
declare(strict_types=1);

/**
 * @template TValue
 * @psalm-yield TValue
 */
interface Promise {}

/**
 * @template TValue
 * @implements Promise<TValue>
 */
final class Success implements Promise {
    /** @param TValue $value */
    public function __construct($value) {
        echo \gettype($value);
    }
}

/** @psalm-yield int */
final class Ticket {}

final class Plain {}

/** @return Promise<string> */
function fetch(): Promise {
    return new Success('x');
}

function takesString(string $value): void {
    echo $value;
}

function takesInt(int $value): void {
    echo $value;
}

function example(): \Generator {
    $data = yield fetch();
    takesString($data);
    takesInt($data);
//           ^^^^^ InvalidArgument: Argument $value of takesInt() expects 'int', got 'string'

    $direct = yield new Success(1);
    /** @mir-check $direct is int */
    takesInt($direct);

    $ticket = yield new Ticket();
    takesInt($ticket);
    takesString($ticket);
//              ^^^^^^^ InvalidArgument: Argument $value of takesString() expects 'string', got 'int'

    $plain = yield new Plain();
    takesInt($plain);
//           ^^^^^^ MixedArgument: Argument $value of takesInt() is mixed

    $bare = yield 1;
    takesInt($bare);
//           ^^^^^ MixedArgument: Argument $value of takesInt() is mixed
}
