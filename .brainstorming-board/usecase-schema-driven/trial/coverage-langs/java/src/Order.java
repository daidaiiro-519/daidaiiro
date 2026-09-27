public class Order {
    public String status = "準備中";
    public java.util.List<String> lines = new java.util.ArrayList<>();

    public void confirm() {
        if (lines.isEmpty()) throw new IllegalStateException("明細が0件");
        status = "確定済";
    }
}
