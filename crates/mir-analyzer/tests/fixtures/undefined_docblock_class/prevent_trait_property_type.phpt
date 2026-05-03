===description===
preventTraitPropertyType
===file===
<?php
                    trait T {}

                    class X {
                      /** @var T|null */
                      public $hm;
                    }
===expect===
UndefinedDocblockClass
===ignore===
TODO
