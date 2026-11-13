===description===
Concatenating lowercase strings (or literals without uppercase letters) gives a lowercase string.
===file===
<?php
final class Ctx {
    /** @var lowercase-string|null */
    public ?string $id = null;
}
function f(Ctx $c, string $cls): void {
    $c->id = strtolower($cls) . '::__construct';
    $c->id = strtolower($cls) . '::Construct';
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidPropertyAssignment: Property $id expects 'lowercase-string|null', cannot assign 'non-empty-string'
}
