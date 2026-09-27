class Order:
    def __init__(self, lines=None):
        self.status = "準備中"
        self.lines = lines or []

    def confirm(self):
        if not self.lines:
            raise ValueError("明細が0件")
        self.status = "確定済"
