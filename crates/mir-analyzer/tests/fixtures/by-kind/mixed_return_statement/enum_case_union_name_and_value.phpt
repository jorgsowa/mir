===description===
`->name` / `->value` on a docblock enum-case union resolve to the enum's name/backing type, also inside seeded callbacks.
===config===
<mir>
  <issueHandlers>
    <MissingClosureReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
enum Suit: string {
    case Hearts = 'h';
    case Spades = 's';
    case Clubs = 'c';

    /** @return list<Suit::Hearts|Suit::Spades> */
    public static function red(): array { return []; }
}

function viaCallbacks(): void {
    array_map(static function (Suit $s): string {
        /** @mir-check $s is Suit::Hearts|Suit::Spades */
        return $s->value;
    }, Suit::red());
    array_map(static function ($s): string {
        /** @mir-check $s is Suit::Hearts|Suit::Spades */
        return $s->name;
    }, Suit::red());
}
