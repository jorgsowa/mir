===description===
A final attribute class used only via #[RouteAttr(...)] on a method and a
method parameter must not be reported UnusedClass.
===config===
suppress=
===file===
<?php
#[Attribute]
final class RouteAttr {
    public function __construct(public string $path = '') {}
}

class Controller {
    #[RouteAttr('/x')]
    public function index(#[RouteAttr('/y')] string $z): void {
        echo $z;
    }
}

(new Controller())->index('a');
===expect===
