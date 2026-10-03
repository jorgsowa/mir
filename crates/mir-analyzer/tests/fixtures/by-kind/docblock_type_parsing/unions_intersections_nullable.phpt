===description===
Union (`A|B|C`), pure intersection (`A&B`), nullable shorthand (`?T`), and
parenthesized combinations (`(A&B)|null`).
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
    <MissingParamType errorLevel="suppress"/>
    <ForbiddenCode errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface Alpha {}
interface Beta {}
class Gamma implements Alpha, Beta {}

function check_union($x) {
    /**
     * @var int|string|bool $x
     * @mir-check $x is int|string|bool
     */
    var_dump($x);
}

function check_nullable_shorthand($x) {
    /**
     * @var ?int $x
     * @mir-check $x is int|null
     */
    var_dump($x);
}

function check_nullable_class($x) {
    /**
     * @var ?Gamma $x
     * @mir-check $x is Gamma|null
     */
    var_dump($x);
}

function check_intersection($x) {
    /**
     * @var Alpha&Beta $x
     * @mir-check $x is Alpha&Beta
     */
    var_dump($x);
}

function check_intersection_three($x) {
    /**
     * @var Alpha&Beta&Gamma $x
     * @mir-check $x is Alpha&Beta&Gamma
     */
    var_dump($x);
}

function check_intersection_with_null($x) {
    /**
     * @var (Alpha&Beta)|null $x
     * @mir-check $x is Alpha&Beta|null
     */
    var_dump($x);
}

function check_union_of_generics($x) {
    /**
     * @var array<int, string>|null $x
     * @mir-check $x is array<int, string>|null
     */
    var_dump($x);
}
===expect===
