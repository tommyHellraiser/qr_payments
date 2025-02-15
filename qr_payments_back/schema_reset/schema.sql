DROP SCHEMA if EXISTS qr_payments;
CREATE SCHEMA qr_payments;
USE qr_payments;

CREATE TABLE `wallets` (
	`ID` INT PRIMARY KEY AUTO_INCREMENT,
	`balance` DECIMAL(12,2) NOT NULL DEFAULT 0
);

CREATE TABLE `transactions` (
	`ID` INT PRIMARY KEY,
	`wallets_ID` INT NULL DEFAULT NULL,
	`amount` DECIMAL(12, 2),
	CONSTRAINT `transactions_wallets_ID` FOREIGN KEY (`wallets_ID`) REFERENCES `wallets` (`ID`) ON UPDATE CASCADE ON DELETE CASCADE
);

INSERT INTO `wallets` (`balance`) VALUES (10000);
INSERT INTO `wallets` (`balance`) VALUES (5000);
