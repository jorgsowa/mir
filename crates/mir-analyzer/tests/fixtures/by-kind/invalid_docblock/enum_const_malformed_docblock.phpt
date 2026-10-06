===description===
FN: an enum constant's own docblock was never validated — only the enum's
own decl docblock was, unlike class/interface/trait members.
===file===
<?php
enum Suit {
    case Hearts;

    /**
     * @var array<>
//     ^^^^^^^^^^^^ InvalidDocblock: Invalid docblock: @var has empty generic type parameter in `array<>`
     */
    const FOO = 1;
}
===expect===
