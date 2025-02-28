package mysql

import (
	"encoding/base64"
	"fmt"
	"strings"
	"time"

	"github.com/inbucket/inbucket/v3/pkg/config"
	"github.com/inbucket/inbucket/v3/pkg/extension"
	"github.com/inbucket/inbucket/v3/pkg/storage"
	"github.com/jhillyerd/enmime/v2"
	"gorm.io/driver/mysql"
	"gorm.io/gorm"
)

// TempEmail 表示临时邮件的数据模型
type TempEmail struct {
	ID        uint32    `gorm:"primaryKey;autoIncrement"`        // 主键,自增
	Email     string    `gorm:"size:50;index:idx_email_created"` // 邮箱地址
	CreatedAt time.Time `gorm:"index:idx_email_created"`         // 创建时间
	Content   string    `gorm:"type:text"`                       // 邮件内容
	Subject   string    `gorm:"size:255"`                        // 邮件主题
	FromEmail string    `gorm:"size:255"`
	FromName  string    `gorm:"size:255"`
}

type Store struct {
	db *gorm.DB
}

func wrapHtml(e *enmime.Envelope) string {
	if len(e.Inlines) == 0 {
		return e.HTML
	}
	for _, inLine := range e.Inlines {
		imgId := "cid:" + inLine.ContentID
		if len(inLine.Content) == 0 {
			continue
		}
		//base64
		replaceImg := "data:application/octet-stream;base64," + base64.StdEncoding.EncodeToString(inLine.Content)
		e.HTML = strings.Replace(e.HTML, imgId, replaceImg, -1)
	}
	return e.HTML
}

// AddMessage implements storage.Store.
func (s *Store) AddMessage(message storage.Message) (id string, err error) {
	// 获取邮件源内容
	r, err := message.Source()
	if err != nil {
		return "", fmt.Errorf("failed to get message source: %v", err)
	}
	defer r.Close()

	// 读取邮件内容
	// content, err := io.ReadAll(r)
	if err != nil {
		return "", fmt.Errorf("failed to read message content: %v", err)
	}
	env, err := enmime.ReadEnvelope(r)
	if err != nil {
		return "", fmt.Errorf("failed to read message content: %v", err)
	}
	// 创建新的临时邮件记录
	tempEmail := &TempEmail{
		Email:     message.To()[0].Address,
		CreatedAt: message.Date(),
		Content:   wrapHtml(env),
		Subject:   message.Subject(),
		FromEmail: message.From().Address,
		FromName:  message.From().Name,
	}

	// 将记录保存到数据库
	result := s.db.Create(tempEmail)
	if result.Error != nil {
		return "", fmt.Errorf("failed to save message to database: %v", result.Error)
	}

	// 返回记录ID作为消息ID
	return fmt.Sprintf("%d", tempEmail.ID), nil
}

// GetMessage implements storage.Store.
func (s *Store) GetMessage(mailbox string, id string) (storage.Message, error) {
	return nil, fmt.Errorf("unimplemented")
}

// GetMessages implements storage.Store.
func (s *Store) GetMessages(mailbox string) ([]storage.Message, error) {
	return nil, fmt.Errorf("unimplemented")
}

// MarkSeen implements storage.Store.
func (s *Store) MarkSeen(mailbox string, id string) error {
	return fmt.Errorf("unimplemented")
}

// PurgeMessages implements storage.Store.
func (s *Store) PurgeMessages(mailbox string) error {
	return fmt.Errorf("unimplemented")
}

// RemoveMessage implements storage.Store.
func (s *Store) RemoveMessage(mailbox string, id string) error {
	return fmt.Errorf("unimplemented")
}

// VisitMailboxes implements storage.Store.
func (s *Store) VisitMailboxes(f func([]storage.Message) (cont bool)) error {
	return nil
}

func New(cfg config.Storage, extHost *extension.Host) (storage.Store, error) {
	//配置MySQL连接参数
	mysqlHost := cfg.MysqlHost
	mysqlPort := 3306
	mysqlUser := cfg.MysqlUsername
	mysqlPass := cfg.MysqlPass
	mysqlDB := "tempemail"

	//构建DSN连接字符串
	dsn := fmt.Sprintf("%s:%s@tcp(%s:%d)/%s?charset=utf8mb4&parseTime=True&loc=Local",
		mysqlUser, mysqlPass, mysqlHost, mysqlPort, mysqlDB)

	//初始化GORM
	db, err := gorm.Open(mysql.Open(dsn), &gorm.Config{})
	if err != nil {
		return nil, fmt.Errorf("failed to connect to database: %v", err)
	}

	// 自动迁移数据库结构
	if err := db.AutoMigrate(&TempEmail{}); err != nil {
		return nil, fmt.Errorf("failed to migrate database: %v", err)
	}

	return &Store{
		db: db,
	}, nil
}
