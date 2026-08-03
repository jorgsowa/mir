===description===
P23 (D8 sibling): Psalm's `PropertyNotSetInConstructor` names the same underlying check
mir reports as `PropertyPossiblyUninitialized` — same aliasing gap as
`PossiblyNullReference`, different target kind. Placed above the constructor (where
mir's own diagnostic actually lands), not the property declaration.
===file===
<?php
class Config {
    public string $env;
    public string $version;

    /** @psalm-suppress PropertyNotSetInConstructor */
    public function __construct(string $env) {
        $this->env = $env;
    }
}
===expect===
