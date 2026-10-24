===description===
A typed static property without a default never needs a constructor; an instance one still does.
===file===
<?php
class Dep {}

class OnlyStatic {
    public static Dep $dep;
    private static int $count;
}

class StaticAndInstance {
//<^^^^^^^^^^^^^^^^^^^^^^^^^ MissingConstructor: Class StaticAndInstance has uninitialized properties but no constructor
    public static int $count;
    public int $id;
}
