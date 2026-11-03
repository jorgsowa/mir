===description===
Private enum methods are checked like class methods
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
enum Suit: string {
    case Hearts = 'H';
    case Spades = 'S';

    public function label(): string {
        return $this->prefix() . $this->value;
    }

    private function prefix(): string {
        return 'suit:';
    }

    private function unused(): void {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnusedMethod: Private method Suit::unused() is never called

    private static function unusedStatic(): void {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnusedMethod: Private method Suit::unusedStatic() is never called
}

echo Suit::Hearts->label();
