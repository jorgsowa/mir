===source===
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
ShadowedTemplateParam: $box->transform('hello')
