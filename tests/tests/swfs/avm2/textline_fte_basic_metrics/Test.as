package {
import flash.display.Sprite;
import flash.text.engine.*;

[SWF(width="200", height="100")]
public class Test extends Sprite {
    public function Test() {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "Liberation Sans";
        fd.fontLookup = FontLookup.DEVICE;
        var fmt:ElementFormat = new ElementFormat(fd, 50);
        var line:TextLine = new TextBlock(new TextElement("Hello", fmt))
            .createTextLine(null, 100000);

        trace("textWidth positive: " + (line.textWidth > 100));
        trace("textHeight positive: " + (line.textHeight > 40));
        trace("ascent positive: " + (line.ascent > 30));
        trace("descent positive: " + (line.descent > 5));
    }
}
}
