===description===
`@template T of static` is checked against the ACTUAL receiver's class
(late static binding), not the class that declares the template. Calling
`accept()` through a `Sub` instance means `static` is `Sub` at this call
site, so passing a plain `Base` (not a `Sub`) violates the bound — even
though `Base` satisfies `T of static` when called through a `Base`
instance directly (see the companion `does_not_report_*` fixture).
===config===
suppress=UnusedParam
===file===
<?php
class Base {
    /**
     * @template T of static
     * @param T $x
     */
    public function accept($x): void {}
}
class Sub extends Base {}

$sub = new Sub();
$sub->accept(new Base());
===expect===
InvalidTemplateParam@12:0-12:24: Template type 'T' inferred as 'Base' does not satisfy bound 'Sub'
