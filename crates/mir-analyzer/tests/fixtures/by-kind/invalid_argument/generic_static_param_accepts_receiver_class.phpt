===description===
`static` nested in a param's generic args accepts generics of the receiver's class hierarchy
===file===
<?php
/** @template T */
class G {}
class H {
    /** @return G<static> */
    public function mk(): G { return new G; }
    /** @param G<static> $g */
    public function take(G $g): void {}
//                       ^^^^ UnusedParam: Parameter $g is never used
    /** @param G<self> $g */
    public function takeSelf(G $g): void {}
//                           ^^^^ UnusedParam: Parameter $g is never used
    public function viaThis(): void {
        $made = $this->mk();
        /** @mir-check $made is G<H> */
        $this->take($made);
    }
}
class Sub extends H {}
/** @param G<Sub> $sub */
function viaSub(Sub $s, G $sub): void {
    $s->take($sub);
    $s->takeSelf($sub);
}
/** @param G<H> $h */
function viaDeclared(H $o, G $h): void {
    /** @mir-check $h is G<H> */
    $o->take($h);
    $o->takeSelf($h);
}
===expect===
