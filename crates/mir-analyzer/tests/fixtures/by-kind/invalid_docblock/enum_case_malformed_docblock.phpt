===description===
FN: an enum case's own docblock was never validated — only the enum's own
decl docblock was, unlike class/interface/trait members.
===file===
<?php
enum Suit {
    /**
     * @var array<>
//     ^^^^^^^^^^^^ InvalidDocblock: Invalid docblock: @var has empty generic type parameter in `array<>`
     */
    case Hearts;
}
===expect===
