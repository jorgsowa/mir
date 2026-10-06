===description===
`cases()`, case-literal unions and `list<Enum::A|Enum::B>` satisfy `list<Enum>` / `list<self>` / `self` / `static` in argument and return positions.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
    <UnnecessaryVarAnnotation errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
enum Suit {
    case A;
    case B;

    /** @return list<self> */
    public static function all(): array { return self::cases(); }

    /** @return list<Suit::A|Suit::B> */
    public static function allCases(): array { return self::cases(); }

    /** @return list<self> */
    public static function fromCaseList(): array { return self::allCases(); }

    /** @return self */
    public static function pick(bool $x) { return $x ? self::A : self::B; }

    /** @return static */
    public static function pickStatic(bool $x) { return $x ? self::A : self::B; }

    /** @param self $s */
    public static function acceptSelf($s): void {}

    public function test(): void {
        $c = self::cases();
        /** @mir-check $c is list<Suit> */
        self::acceptSelf($c[0]);
        self::acceptSelf(self::pick(true));
    }
}

/** @param list<Suit> $x */
function takeList(array $x): void {}

/** @param Suit $x */
function takeEnum($x): void {}

function check(): void {
    takeList(Suit::cases());
    takeList(Suit::allCases());
    /** @var list<Suit::A|Suit::B> $l */
    $l = [];
    takeList($l);
    /** @var Suit::A|Suit::B $u */
    $u = Suit::A;
    takeEnum($u);
    /** @mir-check $l is list<Suit::A|Suit::B> */
    echo 1;
}
