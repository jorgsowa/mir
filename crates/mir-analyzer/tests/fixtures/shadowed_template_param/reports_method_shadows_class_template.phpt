===description===
reports method shadows class template
===file===
<?php
/** @template T */
class Box {
    /**
     * @template T
     * @param T $value
     * @return T
     */
    public function transform($value) { return $value; }
}

function test(): void {
    /** @var Box<string> $box */
    $box = new Box();
    $box->transform('hello');
}
===expect===
ShadowedTemplateParam@15:5: Method template parameter 'T' shadows class-level template parameter with the same name
