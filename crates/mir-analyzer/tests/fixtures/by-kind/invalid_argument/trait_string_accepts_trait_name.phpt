===description===
`trait-string` accepts the name of a trait and rejects the name of a class using it.
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

trait Greets
{
    public function greet(): string
    {
        return 'hi';
    }
}

final class Greeter
{
    use Greets;
}

/** @return trait-string */
function returnsTraitString()
{
    return Greets::class;
}

/** @param trait-string $value */
function acceptsTraitString($value): void {}

function acceptsString(string $value): void {}

acceptsString(returnsTraitString());
acceptsTraitString(Greets::class);
acceptsTraitString(Greeter::class);
//                 ^^^^^^^^^^^^^^ InvalidArgument: Argument $value of acceptsTraitString() expects 'trait-string', got 'class-string<App\Greeter>'
