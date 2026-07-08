===description===
Child redeclares a property with the same static-ness as the parent — valid PHP
===file===
<?php
class A {
    public static int $x = 0;
}

class B extends A {
    public static int $x = 1;
}

class C {
    public int $y = 0;
}

class D extends C {
    public int $y = 1;
}
===expect===
