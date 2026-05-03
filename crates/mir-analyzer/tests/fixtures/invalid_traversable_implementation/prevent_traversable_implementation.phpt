===description===
preventTraversableImplementation
===file===
<?php
                    /**
                     * @implements Traversable<int, int>
                     */
                    final class C implements Traversable {}
                
===expect===
InvalidTraversableImplementation
===ignore===
TODO
