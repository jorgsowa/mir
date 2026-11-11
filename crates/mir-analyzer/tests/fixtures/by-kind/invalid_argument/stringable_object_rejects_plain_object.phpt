===description===
`stringable-object` accepts only objects that can be cast to string.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace App;

final class Money {
    public function __toString(): string {
        return '10 EUR';
    }
}

final class Point {
    public int $x = 0;
}

/** @param stringable-object $value */
function render($value): void {}

render(new Money());
render(new Point());
//     ^^^^^^^^^^^ InvalidArgument: Argument $value of render() expects 'Stringable', got 'App\Point'
render(new \stdClass());
//     ^^^^^^^^^^^^^^^ InvalidArgument: Argument $value of render() expects 'Stringable', got 'stdClass'
render('10 EUR');
//     ^^^^^^^^ InvalidArgument: Argument $value of render() expects 'Stringable', got '"10 EUR"'
