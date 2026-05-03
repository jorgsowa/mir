===description===
circularReference
===file===
<?php
                    class A extends A {}
===expect===
CircularReference
===ignore===
TODO
