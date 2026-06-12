===description===
A method-level @template shadows a same-named class template during argument
checking: the param must bind from the argument, not from the receiver's
class binding (ReflectionClass<Foo>::getAttributes(Attr::class) pattern).
The trailing description containing " of " must not be misparsed as the
template's bound.
===file===
<?php
/** @template T of object */
class Box {
    /**
     * @template T
     *
     * Returns an array of class attributes.
     *
     * @param class-string<T>|null $name
     * @return T|null
     */
    public function pick(?string $name = null) { return null; }
}

class Foo {}
class Attr {}

/** @var Box<Foo> $box */
$box = new Box();
$a = $box->pick(Attr::class);
/** @mir-check $a is Attr|null */
echo $a !== null ? 'y' : 'n';
===expect===
ShadowedTemplateParam@20:6-20:29: Method template parameter 'T' shadows class-level template parameter with the same name
