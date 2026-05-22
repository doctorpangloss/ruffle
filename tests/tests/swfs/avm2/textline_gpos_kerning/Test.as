package {
    import flash.display.Sprite;
    import flash.system.fscommand;
    import flash.text.engine.ElementFormat;
    import flash.text.engine.FontDescription;
    import flash.text.engine.FontLookup;
    import flash.text.engine.TextBlock;
    import flash.text.engine.TextElement;
    import flash.text.engine.TextLine;

    // Pins which kerning source flash.text.engine reads when a font's GPOS
    // and legacy `kern` table disagree. The font GposKernProbe is contrived
    // so the A/V pair carries prime, opposite-sign values in the two tables:
    // GPOS -661 units, `kern` +547 units. At 100pt with 2048 upm, GPOS
    // applied gives delta = -32.275390625, the legacy table gives
    // +26.708984375. Anything else means neither was read.
    public class Test extends Sprite {
        public function Test() {
            var off:TextLine = makeLine("off");
            var on:TextLine = makeLine("on");
            // Round to a quarter pixel so Flash's high-precision textWidth
            // and Ruffle's twip-quantised value agree exactly; the GPOS
            // (-32.25) vs kern-table (+26.75) signal is still unmissable.
            trace("GPOS_PROBE kerning=off textWidth=" + q(off.textWidth));
            trace("GPOS_PROBE kerning=on  textWidth=" + q(on.textWidth));
            trace("GPOS_PROBE delta=" + q(on.textWidth - off.textWidth));

            off.x = 20; off.y = 60 + off.ascent;
            on.x = 20; on.y = 130 + on.ascent;
            addChild(off);
            addChild(on);

            fscommand("quit");
        }

        private function q(n:Number):Number {
            return Math.round(n * 4) / 4;
        }

        private function makeLine(kerning:String):TextLine {
            var fd:FontDescription = new FontDescription();
            fd.fontName = "GposKernProbe";
            fd.fontLookup = FontLookup.DEVICE;
            var ef:ElementFormat = new ElementFormat(fd, 100);
            ef.kerning = kerning;
            var tb:TextBlock = new TextBlock(new TextElement("AV", ef));
            return tb.createTextLine(null, 100000);
        }
    }
}
