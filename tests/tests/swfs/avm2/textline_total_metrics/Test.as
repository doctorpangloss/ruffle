package {
import flash.display.Sprite;
import flash.text.engine.*;

[SWF(width="200", height="100")]
public class Test extends Sprite {
    public function Test() {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "Liberation Sans";
        fd.fontLookup = FontLookup.DEVICE;
        var ef:ElementFormat = new ElementFormat(fd, 20);
        // Plain line for the total* metrics and unjustifiedTextWidth == textWidth.
        var tb:TextBlock = new TextBlock(new TextElement("Hello world", ef));
        var line:TextLine = tb.createTextLine(null, 200);
        trace("totalAscent: " + q(line.totalAscent));
        trace("totalDescent: " + q(line.totalDescent));
        trace("totalHeight: " + q(line.totalHeight));
        trace("unjustifiedTextWidth: " + q(line.unjustifiedTextWidth));
        trace("textWidth: " + q(line.textWidth));
        trace("hasTabs (plain): " + line.hasTabs);
        line.x = 0; line.y = line.ascent;
        addChild(line);
        // Second line just to flip hasTabs.
        var tb2:TextBlock = new TextBlock(new TextElement("a\tb", ef));
        var line2:TextLine = tb2.createTextLine(null, 200);
        trace("hasTabs (tabbed): " + line2.hasTabs);
        line2.x = 0; line2.y = 60;
        addChild(line2);
    }
    private function q(n:Number):Number { return Math.round(n*4)/4; }
}
}
